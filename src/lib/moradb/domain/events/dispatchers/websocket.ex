defmodule Moradb.Events.Dispatchers.Websocket do
  @behaviour Moradb.Events.Dispatcher
  require Logger
  alias Moradb.Event

  @spec dispatch(Event.t()) :: {:ok}
  def dispatch(event) do
    Logger.info("dispatching  event #{event.id}")
    Logger.info("Simulating delay...🕐")
    Process.sleep(1000)
    Logger.info("FIRE 🔥")

    Registry.Moradb
    |> Registry.dispatch(event.category, fn entries ->
      for {pid, _} <- entries do
        if pid != self() do
          Process.send(pid, event, [])
        end
      end
    end)

    {:ok}
  end
end
