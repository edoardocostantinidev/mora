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

    process_events(body)
    send_resp(conn, 200, body)
  end

  defp process_events(events) do
    events = Poison.decode!(events, as: [%Moradb.Event{}])

    events
    |> Enum.map(fn event ->
      event_hash = :erlang.phash2(event)
      Map.put(event, :id, "#{event.createdAt}-#{event.fireAt}-#{event_hash}")
    end)
    |> Enum.each(fn event ->
      Moradb.Events.Database.Local.save(event)
      GenServer.cast(Moradb.Events.TemporalQueue.Local, {:notify, event})
    end)
  end
end
