defmodule Mora.Dispatchers.Websocket do
  @behaviour Mora.Dispatcher

  require Logger
  use GenServer
  alias Mora.Model.Event

  @spec start_link(any) :: :ignore | {:error, any} | {:ok, pid}
  def start_link(_opts) do
    GenServer.start_link(__MODULE__, :ok, name: __MODULE__)
  end

  @doc """
  starts up a websocket dispatcher.
  """
  def init(_) do
    Logger.info("Initializing Websocket dispatcher")
    :ok = :pg.join(__MODULE__, self())
    Logger.info("Websocket dispatcher initialized")
    {:ok, {}}
  end

  def handle_cast({:dispatch, event}, state) do
    dispatch(event)
    {:noreply, state}
  end

  @spec dispatch(Event.t()) :: {:ok}
  def dispatch(event) do
    Logger.info("dispatching event #{event.id}")

    Registry.Mora
    |> Registry.dispatch(event.category, fn entries ->
      for {pid, _} <- entries do
        if pid != self() do
          Process.send(pid, event, [])
        end
      end
    end)

    Logger.info("dispatched event #{event.id}")
    {:ok}
  end
end
