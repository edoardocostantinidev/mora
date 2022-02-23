defmodule Mora.TemporalQueue do
  @moduledoc """
  Priority Temporal queues store events in memory in a priority queue structure where fireAt timestamp is the sort key.
  Module's state is a tuple containing `{current_min, current_max, current_size, pqueue}`. Respectively the current minimum fireAt timestamp, the current maximum fireAt timestamp, the current size and the queue itself.

  Whenever an event is sent here and the queue has space available it will always be enqueued.
  If the queue does not have space then it will check if the event falls in range.
  If it falls in queue's range then it will be enqueued and the last item will be removed.
  Item that do not fall in queue's range will be discarded.

  @moduledoc since: "0.1.0"
  """

  require Logger
  @behaviour Mora.TemporalQueue.Behaviour

  def enqueue(event, state) do
    is_space_available = state.current_size < state.max_size
    is_event_in_range = event.fireAt < state.current_max
    do_enqueue(event, state, is_space_available, is_event_in_range)
  end

  defp do_enqueue(event, state, false, true) do
    new_pq =
      [event | state.queue]
      |> Enum.sort_by(fn e -> e.fireAt end)
      |> Enum.take(state.max_size)

    current_min = get_current_min(new_pq, state.current_min)
    current_max = get_current_max(new_pq, state.current_max)

    %{
      current_min: current_min,
      current_max: current_max,
      max_size: state.max_size,
      current_size: state.current_size,
      queue: new_pq
    }
  end

  defp do_enqueue(event, state, true, _) do
    new_pq =
      [event | state.queue]
      |> Enum.sort_by(fn e -> e.fireAt end)

    current_min = get_current_min(new_pq, state.current_min)
    current_max = get_current_max(new_pq, state.current_max)

    %{
      current_min: current_min,
      current_max: current_max,
      max_size: state.max_size,
      current_size: state.current_size + 1,
      queue: new_pq
    }
  end

  defp do_enqueue(_event, state, _, _) do
    state
  end

  defp get_current_min(pq, min) do
    case(pq) do
      [] ->
        0

      _ ->
        pq
        |> Enum.take(1)
        |> Enum.at(0)
        |> Map.get(:fireAt, min)
    end
  end

  defp get_current_max(pq, max) do
    case(pq) do
      [] ->
        0

      _ ->
        pq
        |> Enum.take(-1)
        |> Enum.at(0)
        |> Map.get(:fireAt, max)
    end
  end
end
