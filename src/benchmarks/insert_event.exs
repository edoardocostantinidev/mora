events_thousand = 0..999
|> Enum.map(fn _ -> Mora.Events.Generator.get_random_event() end)

events_hundred = 0..99_999
|> Enum.map(fn _ -> Mora.Events.Generator.get_random_event() end)


Benchee.run(
  %{
    "insert_thousand_events" => fn ->
      events_thousand
    |> Enum.map(fn event ->
      event_hash = :erlang.phash2(event)
      Map.put(event, :id, "#{event.createdAt}-#{event.fireAt}-#{event_hash}")
    end)
    |> Enum.each(fn event ->
      Mora.Events.Database.Mnesia.save(event)
      GenServer.cast(Mora.Events.TemporalQueue.Priority, {:notify, event})
    end)
    end,
     "insert_one_hundred_thousand_events" => fn ->
      events_hundred
    |> Enum.map(fn event ->
      event_hash = :erlang.phash2(event)
      Map.put(event, :id, "#{event.createdAt}-#{event.fireAt}-#{event_hash}")
    end)
    |> Enum.each(fn event ->
      Mora.Events.Database.Mnesia.save(event)
      GenServer.cast(Mora.Events.TemporalQueue.Priority, {:notify, event})
    end)
    end,
  },
  formatters: [
    Benchee.Formatters.HTML,
    Benchee.Formatters.Console
  ],
  time: 10,
  memory_time: 5,
  parallel: 24,
)
