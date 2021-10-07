defmodule Mora.Events.Router do
  use Plug.Router

  plug(:match)
  plug(:dispatch)

  post "/" do
    {:ok, body, conn} =
      conn
      |> read_body()

    process_events(body)
    send_resp(conn, 200, body)
  end

  defp process_events(events) do
    events = Poison.decode!(events, as: [%Mora.Event{}])

    events
    |> Enum.map(fn event ->
      event_hash = :erlang.phash2(event)
      Map.put(event, :id, "#{event.createdAt}-#{event.fireAt}-#{event_hash}")
    end)
    |> Enum.each(fn event ->
      Mora.Events.Database.Mnesia.save(event)
      Mora.Events.TemporalQueue.Priority.notify(event)
    end)
  end
end
