defmodule Moradb.Events.TemporalQueue do
  @doc """
  temporal queues store events in memory and Whenever a key needs to be dispatched it invokes the relevant dispatcher.
  """
  @callback notify(Moradb.Event.t()) :: :ok | {:error, String.t()}
end
