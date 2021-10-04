defmodule Mora.Events.Dispatchers.Websocket do
  @behaviour Mora.Events.Dispatcher
  require Logger
  alias Mora.Event

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
