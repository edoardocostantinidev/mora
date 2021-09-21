defmodule Moradb.Test.Events.Database do
  use ExUnit.Case
  doctest Moradb
  alias Moradb.Events.Database.Local
  alias Moradb.Events.Generator

  setup _ do
    Memento.Table.clear(Moradb.Event)
  end

  test "save function should save an event to database" do
    event = Generator.get_random_event()
    Local.save(event)
    {:ok, events} = Local.get_all()

    count =
      events
      |> Enum.count()

    assert count == 1
  end

  test "get from function should correctly return filtered events" do
    0..9
    |> Enum.each(fn index ->
      Local.save(Generator.get_random_event())
    end)

    {:ok, events} = Local.get_all()
    IO.inspect(events)
    count = events |> Enum.count()
    assert count == 10
  end
end
