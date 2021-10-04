defmodule Mora.Events.Dispatcher do
  @doc """
  dispatches an event
  """
  @callback dispatch(Mora.Event.t()) :: {:ok} | {:error, String.t()}
end
