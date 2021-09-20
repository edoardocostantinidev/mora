defmodule Moradb.Events.Dispatchers.Websocket do
  @behaviour Moradb.Events.Dispatcher
  require Logger
  alias Moradb.Event

  @spec dispatch(Event.t()) :: {:ok}
  def dispatch(event) do
    Logger.info("dispatching event #{event.id} ⚪")
    Logger.info("FIRE 🔥")

    Registry.Moradb
    |> Registry.dispatch(event.category, fn entries ->
      for {pid, _} <- entries do
        if pid != self() do
          Process.send(pid, event, [])
        end
      end
    end)

    Logger.info("dispatched event #{event.id} 🟢")
    {:ok}
  end
end
