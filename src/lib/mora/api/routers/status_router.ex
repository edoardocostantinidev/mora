defmodule Mora.Api.Routers.Status do
  @moduledoc """
  This module contains the status router and healthchecks.
  """
  use Plug.Router
  alias Mora.TemporalQueue

  plug(:match)
  plug(:dispatch)

  get "/health" do
    send_resp(conn, 200, "OK")
  end

  get "/queues" do
    resp_body =
      :pg.get_members(TemporalQueue.pg_system_name())
      |> Enum.map(fn pid ->
        GenServer.call(pid, :info)
      end)
      |> Poison.encode!()

    send_resp(conn, 200, resp_body)
  end
end
