defmodule Moradb.Events.TemporalQueue.Local do
  @behaviour Moradb.Events.TemporalQueue
  require Logger

  def notify(event) do
    Logger.info("notifying queues about #{event.id}")
    Moradb.Events.Dispatchers.Websocket.dispatch(event)
    {:error, "not implemented"}
  end
end
