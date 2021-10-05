defmodule Mora.Events.Database.Mnesia do
  @behaviour Mora.Events.Database
  use GenServer
  require Logger

  def start_link(_opts) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  def init(_) do
    Logger.debug("Initializing Mnesia")

    if path = Application.get_env(:mnesia, :dir) do
      Logger.debug("Trying to create dir ${path}")
      :ok = File.mkdir_p!(path)
      Logger.debug("Dir ${path} created.")
    end

    nodes = [node() | Node.list()]
    Logger.debug("Retrieved ${nodes}")
    :rpc.multicall(nodes, Memento, :stop, [])
    Memento.Schema.create(nodes)
    Logger.debug("Created schema on ${nodes}")
    :rpc.multicall(nodes, Memento, :start, [])
    Memento.Table.create(Mora.Event, disc_copies: nodes)
    Logger.debug("Created tables on ${nodes}")

    Logger.debug("Initialized Mnesia")
    {:ok, {}}
  end

  def save(event) do
    Logger.debug("saving #{event.id} locally")
    GenServer.cast(__MODULE__, {:save, event})
    Logger.debug("saved #{event.id} locally")

    {:ok}
  end

  def handle_cast({:save, event}, state) do
    Logger.debug("writing event #{event.id} to disk")

    Memento.transaction!(fn ->
      Memento.Query.write(event)
    end)

    Logger.debug("wrote event #{event.id} to disk")
    {:noreply, state}
  end

  def handle_cast(cast, state) do
    Logger.warn("Received a weird cast on #{__MODULE__}: #{cast} #{state}")

    {:noreply, state}
  end

  def get_all() do
    Logger.info("getting all events")
    data = GenServer.call(__MODULE__, {:get})
    {:ok, data}
  end

  def get_from(opts \\ []) do
    timestamp = Keyword.get(opts, :timestamp, -1)
    limit = Keyword.get(opts, :limit, -1)
    Logger.info("getting #{limit} events from #{timestamp} onwards")
    data = GenServer.call(__MODULE__, {:get, timestamp, limit})
    {:ok, data}
  end

  def handle_call({:get}, _from, state) do
    Logger.debug("received get all call")

    events =
      Memento.transaction!(fn ->
        Memento.Query.all(Mora.Event)
      end)

    {:reply, events, state}
  end

  def handle_call({:get, timestamp, limit}, _from, state) do
    Logger.debug("received get call with timestamp: #{timestamp} limit: #{limit}")

    events =
      Memento.transaction!(fn ->
        Memento.Query.all(Mora.Event)
      end)
      |> Enum.filter(fn event -> event.fireAt >= timestamp end)
      |> Enum.take(limit)

    {:reply, events, state}
  end

  def handle_call(call, state) do
    Logger.warn("Received a weird call on #{__MODULE__}: #{call} #{state}")

    {:noreply, state}
  end
end