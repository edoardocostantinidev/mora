defmodule Mora.Events.TemporalQueue.Priority do
  @moduledoc """
  Priority Temporal queues store events in memory in a priority queue structure where fireAt timestamp is the sort key.
  Module's state is a tuple containing `{current_min, current_max, current_size, pqueue}`. Respectively the current minimum fireAt timestamp, the current maximum fireAt timestamp, the current size and the queue itself.

  Whenever an event is sent here and the queue has space available it will always be enqueued.
  If the queue does not have space then it will check if the event falls in range.
  If it falls in queue's range then it will be enqueued and the last item will be removed.
  Item that do not fall in queue's range will be discarded.

  @moduledoc since: "0.1.0"
  """

  @behaviour Mora.Events.TemporalQueue
  require Logger
  use GenServer
  @max_size 1000
  @tick 999

  @spec start_link(any) :: :ignore | {:error, any} | {:ok, pid}
  def start_link(_opts) do
    GenServer.start_link(__MODULE__, :ok, name: __MODULE__)
  end

  @doc """
  starts up a temporal queue with priority implementation.
  """
  @spec init(any) :: {:ok, {0, 0, 0, []}}
  def init(_) do
    Logger.info("Initializing TemporalQueue")
    schedule_tick()
    pqueue = []
    current_min = 0
    current_max = 0
    current_size = 0
    {:ok, {current_min, current_max, current_size, pqueue}}
  end

  @doc """
  main cast handler, available options are:

  - `:tick` handles `tick` casts. Each tick represents the delta-t the temporal queue loops around.
  If @tick is set to 999 then the queue will try to dispatch event every 999 ms.
  It's basically the time resolution of Mora.
  - `:clear` clears the queue.
  - `{:notify,event}` handles notification casts. Each event is notified to the queue so it can be enqueued or discarded.

  """
  def handle_cast(:tick, state) do
    t1 = :erlang.system_time(:microsecond)
    new_state = do_tick(state)
    t2 = :erlang.system_time(:microsecond)
    Logger.info("Tick performed in #{t2 - t1}µs")
    {:noreply, new_state}
  end

  def handle_cast({:notify, event}, state) do
    {:ok, new_state} = notify(event, state)
    {:noreply, new_state}
  end

  def handle_cast(:clear, _state) do
    pqueue = []
    current_min = 0
    current_max = 0
    current_size = 0
    {:noreply, {current_min, current_max, current_size, pqueue}}
  end

  def handle_cast(msg, state) do
    Logger.warn("Received a weird message on temporal queue:\n#{msg} ")

    Logger.warn("Ignoring ")
    {:noreply, state}
  end

  def handle_call(:info, _from, state) do
    {min, max, size, _pq} = state

    {:reply,
     %{
       queue_size: size,
       queue_temporal_min: min,
       queue_temporal_max: max
     }, state}
  end

  @doc """
  returns the max_size of this queue
  """
  def max_size, do: @max_size

  def notify(event, state) do
    {_min, max, size, _pq} = state

    is_space_available = size < @max_size
    is_event_in_range = event.fireAt < max

    Logger.debug(
      "Handling :notify event: #{event.id} for #{event.category}.\nSpace Available in queue:#{@max_size - size}\nEvent is in range: #{is_event_in_range}"
    )

    {:ok, enqueue(event, state, is_space_available, is_event_in_range)}
  end

  defp enqueue(event, state, false, true) do
    {min, max, size, pq} = state

    new_pq =
      [event | pq]
      |> Enum.sort_by(fn e -> e.fireAt end)
      |> Enum.take(@max_size)

    current_min = get_current_min(new_pq, min)
    current_max = get_current_max(new_pq, max)

    {current_min, current_max, size, new_pq}
  end

  defp enqueue(event, state, true, _) do
    {min, max, size, pq} = state

    new_pq =
      [event | pq]
      |> Enum.sort_by(fn e -> e.fireAt end)

    current_min = get_current_min(new_pq, min)
    current_max = get_current_max(new_pq, max)

    {current_min, current_max, size + 1, new_pq}
  end

  defp enqueue(_event, state, _, _) do
    state
  end

  defp get_current_min(pq, min) do
    case(pq) do
      [] ->
        0

      _ ->
        pq
        |> Enum.take(1)
        |> Enum.at(0)
        |> Map.get(:fireAt, min)
    end
  end

  defp get_current_max(pq, max) do
    case(pq) do
      [] ->
        0

      _ ->
        pq
        |> Enum.take(-1)
        |> Enum.at(0)
        |> Map.get(:fireAt, max)
    end
  end

  defp do_tick(state) do
    {min, max, size, pq} = state
    time = :os.system_time(:millisecond)
    Logger.debug("Handling :tick #{time}\nMin:#{min} Max:#{max} QueueSize: #{size}")

    consumed_pq =
      pq
      |> Enum.filter(fn event -> event.fireAt <= time end)

    consumed_pq
    |> Enum.each(fn event ->
      :pg.get_members(Mora.Events.Dispatchers.Websocket)
      |> Enum.each(fn pid -> GenServer.cast(pid, {:dispatch, event}) end)
    end)

    new_pq = pq -- consumed_pq
    schedule_tick()

    max_events_to_retrieve = Enum.count(consumed_pq)
    current_max = get_current_max(consumed_pq, max)
    new_size = Enum.count(new_pq)

    if Enum.count(consumed_pq) > 0 && size == @max_size do
      GenServer.call(Mora.Events.Database.Mnesia, {:get, current_max + 1, max_events_to_retrieve})
      |> Enum.each(fn event -> GenServer.cast(self(), {:notify, event}) end)
    end

    {min, max, new_size, new_pq}
  end

  defp schedule_tick() do
    Logger.debug("Scheduling Tick")
    self_pid = self()

    Task.start(fn ->
      :timer.sleep(@tick)
      GenServer.cast(self_pid, :tick)
    end)

    Logger.debug("Done")
  end
end
