defmodule Mora.Api.Routers.Status do
  use Plug.Router

  plug(:match)
  plug(:dispatch)

  get "/health" do
    send_resp(conn, 200, "OK")
  end

  get "/queue" do
    resp_body =
      GenServer.call(Mora.TemporalQueue.Priority, :info)
      |> Poison.encode!()

    send_resp(conn, 200, resp_body)
  end
end
