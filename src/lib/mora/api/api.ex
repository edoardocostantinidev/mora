defmodule Mora.Api do
  use Plug.Router
  use PlugSocket
  socket("/ws/events/[...]", Mora.Api.SocketHandler.Event)
  plug(:match)
  plug(:dispatch)
  forward("/events", to: Mora.Api.Routers.Event)

  get "/health" do
    send_resp(conn, 200, "OK")
  end
end
