defmodule Moradb.Events.TemporalQueue.Priority do
  @doc """
  Priority Temporal queues store events in memory in a priority queue structure where fireAt timestamp is the sort key.
  """

  @moduledoc """

  """
  @behaviour Moradb.Events.TemporalQueue
  require Logger
  use GenServer
  @max_size 1000
  @tick 999

  def start_link(_opts) do
    GenServer.start_link(__MODULE__, :ok, name: __MODULE__)
  end

  def init(_) do
    Logger.info("Initializing TemporalQueue")
    schedule_tick()
    pqueue = []
    current_min = 0
    current_max = 0
    current_size = 0
    {:ok, {current_min, current_max, current_size, pqueue}}
  end

  def handle_info(msg, state) do
    GenServer.cast(__MODULE__, msg)
    {:noreply, state}
  end

  def handle_cast(:tick, state) do
    {min, max, size, pq} = state
    time = :os.system_time(:millisecond)
    Logger.debug("Handling :tick #{time}\nMin:#{min} Max:#{max} QueueSize: #{size}")

    consumed_pq =
      pq
      |> Enum.filter(fn event -> event.fireAt <= time end)

    consumed_pq
    |> Enum.each(fn event ->
      Moradb.Events.Dispatchers.Websocket.dispatch(event)
    end)

    new_pq = pq -- consumed_pq
    schedule_tick()
    {:noreply, {min, max, Enum.count(new_pq), new_pq}}
  end

  def handle_cast({:notify, event}, state) do
    {_min, max, size, _pq} = state

    is_space_available = size < @max_size
    is_event_in_range = event.fireAt < max

    Logger.debug(
      "Handling :notify event: #{event.id} for #{event.category}.\nSpace Available in queue:#{@max_size - size}\nEvent is in range: #{is_event_in_range}"
    )

    new_state = enqueue(event, state, is_space_available, is_event_in_range)
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

  def notify(event) do
    Logger.debug("Notifying queues about #{event.id}")
    GenServer.cast(__MODULE__, {:notify, event})
  end

  def max_size, do: @max_size

  defp enqueue(event, state, false, true) do
    {min, max, size, pq} = state

    new_pq =
      [event | pq]
      |> Enum.sort_by(fn e -> e.fireAt end)
      |> Enum.take(@max_size)

    current_min =
      new_pq
      |> Enum.at(0)
      |> Map.get(:fireAt, min)

    current_max =
      new_pq
      |> Enum.at(Enum.count(new_pq) - 1)
      |> Map.get(:fireAt, max)

    {current_min, current_max, size, new_pq}
  end

  defp enqueue(event, state, true, _) do
    {min, max, size, pq} = state

    new_pq =
      [event | pq]
      |> Enum.sort_by(fn e -> e.fireAt end)

    current_min =
      new_pq
      |> Enum.at(0)
      |> Map.get(:fireAt, min)

    current_max =
      new_pq
      |> Enum.at(Enum.count(new_pq) - 1)
      |> Map.get(:fireAt, max)

    {current_min, current_max, size + 1, new_pq}
  end

  defp enqueue(_event, state, _, _) do
    state
  end

  defp schedule_tick() do
    Logger.debug("Scheduling Tick")
    Process.send_after(self(), :tick, @tick)
    Logger.debug("Done")
  end
end
