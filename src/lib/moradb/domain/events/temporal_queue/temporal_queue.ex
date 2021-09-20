defmodule Moradb.Events.TemporalQueue do
  @doc """
  temporal queues store events in memory in a priority queue structure where fireAt timestamp is the sort key.
  Whenever a key needs to be dispatched the temporal queue does it by invoking the relevant dispatcher.
  """
  @callback notify(Moradb.Event.t()) :: :ok | {:error, String.t()}
end
