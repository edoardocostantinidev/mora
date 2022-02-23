defmodule Mora.TemporalQueue.Server do
  @moduledoc """
  Priority Temporal queues store events in memory in a priority queue structure where fireAt timestamp is the sort key.
  Module's state is a tuple containing `{current_min, current_max, current_size, pqueue}`. Respectively the current minimum fireAt timestamp, the current maximum fireAt timestamp, the current size and the queue itself.

  Whenever an event is sent here and the queue has space available it will always be enqueued.
  If the queue does not have space then it will check if the event falls in range.
  If it falls in queue's range then it will be enqueued and the last item will be removed.
  Item that do not fall in queue's range will be discarded.

  @moduledoc since: "0.1.0"
  """
  @max_size Application.get_env(:mora, :max_size, 1000)
  @tick Application.get_env(:mora, :tick, 100)
  @pg_name "temporal_queues"
  @pg_system_name "system:temporal_queues"
  @behaviour Mora.CommonBehaviour.PgItem

  alias Mora.TemporalQueue
  require Logger
  use GenServer

  @spec start_link(any) :: :ignore | {:error, any} | {:ok, pid}
  def start_link(%{category: category}) do
    GenServer.start_link(__MODULE__, {:ok, category}, name: __MODULE__)
  end

  @doc """
  starts up a temporal queue with priority implementation.
  """

  def init({:ok, category}) do
    :pg.join(@pg_system_name, self())
    :pg.join(pg_name(category), self())
    schedule_tick()
    pqueue = []
    current_min = 0
    current_max = 0
    current_size = 0
    max_size = @max_size

    {:ok,
     %{
       current_min: current_min,
       current_max: current_max,
       current_size: current_size,
       max_size: max_size,
       queue: pqueue
     }}
  end

  @doc """
  main cast handler, available options are:

  - `:tick` handles `tick` casts. Each tick represents the delta-t the temporal queue loops around.
  If @tick is set to 999 then the queue will try to dispatch event every 999 ms.
  It's basically the time resolution of Mora.
  - `:clear` clears the queue.
  - `{:notify,event}` handles notification calls. Each event is notified to the queue so it can be enqueued or discarded.

  """
  def handle_info(:tick, state) do
    new_state = do_tick(state)

    {:noreply, new_state}
  end

  def handle_call({:notify, event}, _, state) do
    Logger.debug(
      "Handling :notify event: #{event.id} for #{event.category}.\nSpace Available in queue:#{state.max_size - state.current_size}"
    )

    new_state = TemporalQueue.enqueue(event, state)
    {:reply, :ok, new_state}
  end

  def handle_call({:unschedule, id}, _, state) do
    Logger.debug("Handling :unschedule event: #{id}")

    new_queue =
      state.queue
      |> Enum.filter(fn event -> event.id != id end)

    new_state =
      state
      |> Map.put(:queue, new_queue)
      |> Map.put(:current_size, Enum.count(new_queue))

    {:reply, :ok, new_state}
  end

  def handle_call(:info, _from, state) do
    {:reply,
     %{
       queue_size: state.current_size,
       queue_temporal_min: state.current_min,
       queue_temporal_max: state.current_max
     }, state}
  end

  def handle_cast(:clear, _state) do
    pqueue = []
    current_min = 0
    current_max = 0
    current_size = 0
    {:stop, :clear, {current_min, current_max, current_size, pqueue}}
  end

  def handle_cast(msg, state) do
    Logger.warn("Received a weird message on temporal queue:\n#{inspect(msg)}")
    {:noreply, state}
  end

  @doc """
  returns the max_size of this queue
  """
  def max_size, do: @max_size

  defp do_tick(state) do
    time = :os.system_time(:millisecond)

    consumed_pq =
      state.queue
      |> Enum.filter(fn event -> event.fireAt <= time end)

    consumed_pq
    |> Enum.each(fn event ->
      event_corrected = Map.put_new(event, :dispatched_from, node())

      event.category
      |> Mora.Dispatchers.Websocket.pg_name()
      |> :pg.get_members()
      |> Enum.each(fn pid ->
        GenServer.cast(pid, {:dispatch, event_corrected})
      end)
    end)

    new_pq = state.queue -- consumed_pq
    schedule_tick()

    max_events_to_retrieve = Enum.count(consumed_pq)
    current_max = state.current_max
    new_size = Enum.count(new_pq)

    if Enum.count(consumed_pq) > 0 && state.current_size == @max_size do
      GenServer.call(Mora.Database.Mnesia, {:get, current_max + 1, max_events_to_retrieve})
      |> Enum.each(fn event -> get_queue_manager().notify(event) end)
    end

    state
    |> Map.put(:current_size, new_size)
    |> Map.put(:queue, new_pq)
  end

  defp schedule_tick(), do: Process.send_after(self(), :tick, @tick)

  defp get_queue_manager(),
    do: Application.get_env(:mora, :temporal_queue_manager, Mora.TemporalQueue.Manager)

  def pg_name(category), do: @pg_name <> ":" <> category
  def pg_system_name(), do: @pg_system_name
end
