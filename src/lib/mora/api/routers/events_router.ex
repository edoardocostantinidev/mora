defmodule Mora.Api.Routers.Event do
  @moduledoc """
  This module contains the event router.
  """
  use Plug.Router

  plug(:match)
  plug(:dispatch)

  post "/" do
    {:ok, body, conn} =
      conn
      |> read_body()

    body
    |> parse_events()
    |> get_events_service().process_events()

    send_resp(conn, 200, body)
  end

  defp parse_events(json) do
    events = Poison.decode!(json, as: [%Mora.Model.Event{}])
  end

  defp get_events_service(), do: Application.get_env(:mora, :events_service, Mora.Service.Events)
end
