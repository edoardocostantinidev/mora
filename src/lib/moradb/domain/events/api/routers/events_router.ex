defmodule Moradb.Events.Router do
  use Plug.Router

  plug(:match)
  plug(:dispatch)

  get "/" do
    conn
    |> send_resp(200, "OK")
  end

  post "/" do
    {:ok, body, conn} =
      conn
      |> read_body()

    IO.inspect(body)
    process_events(body)

    send_resp(conn, 200, body)
  end

  defp process_events(events) do
    events = Poison.decode!(events, as: [%Moradb.Event{}])

    events
    |> Enum.each(fn event ->
      Moradb.Events.Dispatchers.Websocket.dispatch(event)
    end)
  end

  defp missing_events do
    Poison.encode!(%{response: "no valid event found!"})
  end
end
