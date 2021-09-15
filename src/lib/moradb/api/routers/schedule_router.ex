defmodule Moradb.Routers.Events do
  use Plug.Router
  plug(:match)
  plug(Plug.Parsers, parsers: [:json], json_decoder: Poison)
  plug(:dispatch)

  get "/" do
    conn
    |> send_resp(200, "test")
  end

  post "/" do
    {status, body} =
      case conn.body_params do
        %{"events" => events} -> {200, process_events(events)}
        _ -> {422, missing_events()}
      end

    send_resp(conn, status, body)
    {:ok}
  end

  defp process_events(events) do
    Poison.encode!(events)
  end

  defp missing_events do
    Poison.encode!(%{response: "no valid event found!"})
  end
end
