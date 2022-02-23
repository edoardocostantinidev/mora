defmodule Mora.Test.TemporalQueue do
  @moduledoc """
  This module tests the temporal queue.
  """

  use ExUnit.Case, async: true
  doctest Mora
  alias Mora.TemporalQueue
  alias Mora.Support.Generator
  require Logger

  test "pqueue should not insert new item when notified of an event and the pqueue doesn't have space and the event is not in range" do
    event_not_in_range = Generator.get_random_event(1000, 2000)
    events_in_queue = 1..5 |> Enum.map(fn _ -> Generator.get_random_event(500, 1000) end)

    state = %{
      current_min: 500,
      current_max: 1000,
      current_size: 5,
      max_size: 5,
      queue: events_in_queue
    }

    new_state = TemporalQueue.enqueue(event_not_in_range, state)
    assert new_state == state
  end

  test "pqueue should insert new item when notified of an event and the pqueue doesn't have space and the event is in range" do
    event_in_range = Generator.get_random_event(500, 502)
    events_in_queue = 1..5 |> Enum.map(fn _ -> Generator.get_random_event(500, 1000) end)

    state = %{
      current_min: 500,
      current_max: 1000,
      current_size: 5,
      max_size: 5,
      queue: events_in_queue
    }

    new_state = TemporalQueue.enqueue(event_in_range, state)
    in_queue = event_in_range in new_state.queue

    assert Enum.count(state.queue) == Enum.count(new_state.queue)
    assert state.current_size == new_state.current_size
    assert in_queue == true
  end
end
