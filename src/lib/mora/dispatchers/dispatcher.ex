defmodule Mora.Dispatcher do
  @doc """
  dispatches an event
  """
  @callback dispatch(Mora.Model.Event.t()) :: {:ok} | {:error, String.t()}
end
