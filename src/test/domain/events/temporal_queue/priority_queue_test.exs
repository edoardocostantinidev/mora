defmodule Moradb.Test.Events.TemporalQueue.Priority do
  use ExUnit.Case
  doctest Moradb
  alias Moradb.Events.TemporalQueue.Priority
  alias Moradb.Events.Generator

  setup _ do
    Memento.Table.clear(Moradb.Event)
  end

  test "queues should not contain more than @max_size events" do
    start_supervised(Priority)

    0..1000
    |> Enum.each(fn _ ->
      event = Generator.get_random_event()
      GenServer.cast(Priority, {:notify, event})
    end)

    info = GenServer.call(Priority, {:info})
    IO.inspect(info)
    %{queue_size: queue_size} = info
    assert queue_size == 1000
  end
end
