defmodule Mora.Test.Events.TemporalQueue.Priority do
  use ExUnit.Case
  doctest Mora
  alias Mora.Events.TemporalQueue.Priority
  alias Mora.Events.Generator

  setup _ do
    Memento.Table.clear(Mora.Event)
    start_supervised(Priority)
    on_exit(fn -> GenServer.cast(Priority, :clear) end)
    :ok
  end

  test "pqueue should insert new item when notified of an event and the pqueue has space available" do
    event = Generator.get_random_event()
    GenServer.cast(Priority, {:notify, event})

    %{queue_size: queue_size, queue_temporal_min: min, queue_temporal_max: max} =
      GenServer.call(Priority, :info)

    assert queue_size == 1
    assert min == event.fireAt
    assert max == event.fireAt
  end

  test "pqueue should not insert new item when notified of an event and the pqueue doesn't have space and the event is not in range" do
    max_size = Priority.max_size()

    1..max_size
    |> Enum.each(fn _ ->
      event = Generator.get_random_event(9_000_000_000, 9_900_000_000)
      GenServer.cast(Priority, {:notify, event})
    end)

    event_outside_range = Generator.get_random_event(9_990_000_000, 9_999_000_000)
    GenServer.cast(Priority, {:notify, event_outside_range})

    %{queue_size: queue_size} = GenServer.call(Priority, :info)

    assert queue_size == max_size
  end

  test "pqueue should insert new item when notified of an event and the pqueue doesn't have space and the event is in range" do
    max_size = Priority.max_size()

    1..max_size
    |> Enum.each(fn _ ->
      event = Generator.get_random_event(9_000_000_000, 9_900_000_000)
      GenServer.cast(Priority, {:notify, event})
    end)

    event_inside_range = Generator.get_random_event(8_000_000_000, 8_900_000_000)
    GenServer.cast(Priority, {:notify, event_inside_range})

    %{queue_size: queue_size} = GenServer.call(Priority, :info)

    assert queue_size == max_size
  end
end
