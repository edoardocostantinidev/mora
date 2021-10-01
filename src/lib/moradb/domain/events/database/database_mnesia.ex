defmodule Moradb.Events.Database.Mnesia do
  @behaviour Moradb.Events.Database
  use GenServer
  require Logger

  def start_link(_opts) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  def init(_) do
    Logger.debug("Initializing DB ⚪")

    if path = Application.get_env(:mnesia, :dir) do
      :ok = File.mkdir_p!(path)
    end

    nodes = [node()]
    Memento.stop()
    Memento.Schema.create(nodes)
    Memento.start()
    Memento.Table.create(Moradb.Event, disc_copies: nodes)
    Logger.debug("Initialized DB 🟢")
    {:ok, {}}
  end

  def save(event) do
    Logger.debug("saving #{event.id} locally ⚪")
    GenServer.cast(__MODULE__, {:save, event})
    Logger.debug("saved #{event.id} locally 🟢")

    {:ok}
  end

  def handle_cast({:save, event}, state) do
    Logger.debug("writing event #{event.id} to disk ⚪")

    Memento.transaction!(fn ->
      Memento.Query.write(event)
    end)

    Logger.debug("wrote event #{event.id} to disk 🟢")
    {:noreply, state}
  end

  def handle_cast(cast, state) do
    Logger.warn(
      "Received a weird cast on #{__MODULE__}: #{IO.inspect(cast)} #{IO.inspect(state)}🟡"
    )

    {:noreply, state}
  end

  def get_all() do
    Logger.info("getting all events ⚪")
    data = GenServer.call(__MODULE__, {:get})
    {:ok, data}
  end

  def get_from(opts \\ [limit: 100, timestamp: -1]) do
    timestamp = opts[:timestamp]
    limit = opts[:limit]
    Logger.info("getting #{limit} events from #{timestamp} onwards ⚪")
    data = GenServer.call(__MODULE__, {:get, timestamp, limit})
    {:ok, data}
  end

  def handle_call({:get}, _from, state) do
    Logger.debug("received get all call ⚪")

    events =
      Memento.transaction!(fn ->
        Memento.Query.all(Moradb.Event)
      end)

    {:reply, events, state}
  end

  def handle_call({:get, timestamp, limit}, _from, state) do
    Logger.debug("received get call with timestamp: #{timestamp} limit: #{limit} ⚪")

    events =
      Memento.transaction!(fn ->
        Memento.Query.all(Moradb.Event)
      end)
      |> Enum.filter(fn event -> event.fireAt >= timestamp end)
      |> Enum.take(limit)

    {:reply, events, state}
  end

  def handle_call(call, state) do
    Logger.warn(
      "Received a weird call on #{__MODULE__}: #{IO.inspect(call)} #{IO.inspect(state)}🟡"
    )

    {:noreply, state}
  end
end
