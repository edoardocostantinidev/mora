defmodule Mora.Events.TemporalQueue do
  @doc """
  temporal queues store events in memory and Whenever a key needs to be dispatched it invokes the relevant dispatcher.
  """
  @callback notify(Mora.Event.t()) :: :ok | {:error, String.t()}
end
