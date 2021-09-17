defmodule Moradb.Api do
  use Plug.Router
  use PlugSocket
  socket("/ws/events/[...]", Moradb.Events.SocketHandler)
  plug(:match)
  plug(:dispatch)
  forward("/events", to: Moradb.Events.Router)

  get "/health" do
    send_resp(conn, 200, "OK")
  end
end
