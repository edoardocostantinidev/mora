defmodule Moradb.Events.Database.Local do
  @behaviour Moradb.Events.Database
  use GenServer
  require Logger
  @db_path ".db"

  def init(_) do
    Logger.debug("Initializing DB ⚪")

    case File.exists?(@db_path) do
      false -> File.touch(@db_path)
      _ -> nil
    end

    file_stream = File.open!(@db_path)
    Logger.debug("Initialized DB 🟢")

    {:ok, {file_stream}}
  end

  def start_link(_opts) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  def save(event) do
    Logger.debug("saving #{event.id} locally ⚪")
    GenServer.cast(__MODULE__, {:save, event})
    Logger.debug("saved #{event.id} locally 🟢")

    {:ok}
  end

  def get_all() do
    Logger.info("getting all events ⚪")
    data = GenServer.call(__MODULE__, {:get})
    {:ok, data}
  end

  def get_from(timestamp \\ -1, limit \\ 100) do
    Logger.info("getting #{limit} events from #{timestamp} onwards ⚪")
    data = GenServer.call(__MODULE__, {:get, timestamp, limit})
    {:ok, data}
  end

  def handle_cast({:save, event}, state) do
    Logger.debug("writing event #{event.id} to disk ⚪")
    {file_stream} = state

    bytes = :erlang.term_to_binary(event)

    file_stream
    |> IO.binwrite(bytes)

    Logger.debug("wrote event #{event.id} to disk 🟢")
    {:noreply, state}
  end

  def handle_cast(cast, state) do
    Logger.warn(
      "Received a weird cast on #{__MODULE__}: #{IO.inspect(cast)} #{IO.inspect(state)}🟡"
    )

    {:noreply, state}
  end

  def handle_call({:get, timestamp, limit}, _from, state) do
    Logger.debug("received get call with timestamp: #{timestamp} limit: #{limit} ⚪")
    {file_stream} = state

    events =
      file_stream
      |> IO.binread(:all)
      |> :erlang.binary_to_term()

    {:reply, events, state}
  end

  def handle_call(call, state) do
    Logger.warn(
      "Received a weird call on #{__MODULE__}: #{IO.inspect(call)} #{IO.inspect(state)}🟡"
    )

    {:noreply, state}
  end
end
