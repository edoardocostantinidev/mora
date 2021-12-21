defmodule Mora.Dispatcher do
  @moduledoc """
  Behaviour for dispatching events to the appropriate handlers.
  """

  @doc """
  dispatches an event
  """
  @callback dispatch(Mora.Model.Event.t()) :: {:ok} | {:error, String.t()}
end
