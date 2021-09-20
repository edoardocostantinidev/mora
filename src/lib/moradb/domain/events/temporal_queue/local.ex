defmodule Moradb.Events.TemporalQueue.Local do
  @behaviour Moradb.Events.TemporalQueue
  require Logger
  use GenServer
  @max_size 1000
  @tick 999

  def start_link(_opts) do
    GenServer.start_link(__MODULE__, :ok, name: __MODULE__)
  end

  def init(_) do
    Logger.debug("Initializing TemporalQueue ⚪")
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
    Logger.info("Handling :tick #{time}\nMin:#{min} Max:#{max} QueueSize: #{size} ⚪")

    consumed_pq =
      pq
      |> Enum.filter(fn event -> event.fireAt <= time end)

    consumed_pq
    |> Enum.each(fn event ->
      Moradb.Events.Dispatchers.Websocket.dispatch(event)
    end)

    new_pq = pq -- consumed_pq

    IO.inspect(new_pq)
    schedule_tick()
    {:noreply, {min, max, Enum.count(new_pq), new_pq}}
  end

  def handle_cast({:notify, event}, state) do
    {min, max, size, pq} = state

    is_space_available = size < @max_size
    is_event_in_range = event.fireAt <= max && event.fireAt >= min

    Logger.debug(
      "Handling :notify event: #{event.id} for #{event.category}.\nSpace Available in queue:#{@max_size - size}\nEvent is in range: #{is_event_in_range} ⚪"
    )

    new_pq = [event | pq]

    IO.inspect(new_pq)

    new_pq =
      case {is_space_available, is_event_in_range} do
        {false, true} ->
          new_pq
          |> Enum.take(@max_size)

        _ ->
          new_pq
      end

    IO.inspect(new_pq)

    current_min =
      new_pq
      |> Enum.at(0)
      |> Map.get(:fireAt, min)

    current_max =
      new_pq
      |> Enum.at(size)
      |> Map.get(:fireAt, max)

    {:noreply, {current_min, current_max, size + 1, new_pq}}
  end

  def handle_cast(msg, state) do
    Logger.warn("Received a weird message on temporal queue 🟡")
    IO.inspect(msg)
    IO.inspect(state)
    Logger.warn("Ignoring 🟡")
    {:noreply, state}
  end

  def notify(event) do
    Logger.debug("Notifying queues about #{event.id} ⚪")
    IO.inspect(self())

    GenServer.cast(__MODULE__, {:notify, event})
  end

  defp schedule_tick() do
    Logger.debug("Scheduling Tick ⚪")
    Process.send_after(self(), :tick, @tick)
    Logger.debug("Done 🟢")
  end
end
