defmodule Mora.Api do
  use Plug.Router
  use PlugSocket
  socket("/ws/events/[...]", Mora.Api.SocketHandler)
  plug(:match)
  plug(:dispatch)
  forward("/events", to: Mora.Api.Router)

  get "/health" do
    send_resp(conn, 200, "OK")
  end
end
