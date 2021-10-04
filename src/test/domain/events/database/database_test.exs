defmodule Mora.Test.Events.Database do
  use ExUnit.Case
  doctest Mora
  alias Mora.Events.Database.Mnesia
  alias Mora.Events.Generator

  setup _ do
    Memento.Table.clear(Mora.Event)
  end

  test "save function should save an event to database" do
    event = Generator.get_random_event()
    Mnesia.save(event)
    {:ok, events} = Mnesia.get_all()

    count =
      events
      |> Enum.count()

    assert count == 1
  end

  test "get all function should return all events" do
    0..9
    |> Enum.each(fn _ ->
      Mnesia.save(Generator.get_random_event())
    end)

    {:ok, events} = Mnesia.get_all()

    count = events |> Enum.count()
    assert count == 10
  end

  test "get from function with timestamp should return filtered events" do
    event1 = Generator.get_random_event(1_000, 1_100)
    event2 = Generator.get_random_event(1_000_000, 1_100_000)
    Mnesia.save(event1)
    Mnesia.save(event2)

    {:ok, events} = Mnesia.get_from(timestamp: 1_101)
    count = events |> Enum.count()
    assert count == 1
  end

  test "get from function with limit should return at most {limit} events" do
    event1 = Generator.get_random_event(1_000, 1_100)
    event2 = Generator.get_random_event(1_000_000, 1_100_000)
    Mnesia.save(event1)
    Mnesia.save(event2)
    {:ok, events} = Mnesia.get_from(limit: 1)
    count = events |> Enum.count()
    assert count == 1
  end
end
