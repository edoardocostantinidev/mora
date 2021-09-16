defmodule Moradb.Events.Dispatcher do
  @doc """
  dispatches an event
  """
  @callback dispatch(Moradb.Event.t()) :: {:ok} | {:error, String.t()}
end
