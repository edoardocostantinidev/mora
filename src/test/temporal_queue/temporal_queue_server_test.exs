defmodule Mora.Test.TemporalQueue.Server do
  @moduledoc """
  This module tests the temporal queue.
  """

  use ExUnit.Case, async: true
  doctest Mora

  alias Mora.TemporalQueue.Server
  alias Mora.Support.Generator
  require Logger

  setup _ do
    Memento.Table.clear(Mora.Model.Event)
    {:ok, _} = start_supervised({Server, %{category: "test"}})
    :ok
  end

  test "pqueue should insert new item when notified of an event and the pqueue has space available" do
    event = Generator.get_random_event()
    GenServer.call(Server, {:notify, event})

    %{queue_size: queue_size, queue_temporal_min: min, queue_temporal_max: max} =
      GenServer.call(Server, :info)

    assert queue_size == 1
    assert min == event.fireAt
    assert max == event.fireAt
  end

  test "pqueue should return correct info when called with :info" do
    event = Generator.get_random_event()
    GenServer.call(Server, {:notify, event})

    %{
      queue_size: queue_size,
      queue_temporal_min: min,
      queue_temporal_max: max
    } = GenServer.call(Server, :info)

    assert queue_size == 1
    assert min == event.fireAt
    assert max == event.fireAt
  end

  test "unschedule should remove event from pqueue" do
    event = Generator.get_random_event()
    GenServer.call(Server, {:notify, event})

    %{queue_size: queue_size} = GenServer.call(Server, :info)

    assert queue_size == 1

    GenServer.call(Server, {:unschedule, event.id})

    %{queue_size: queue_size} = GenServer.call(Server, :info)

    assert queue_size == 0
  end
end
