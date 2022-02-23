defmodule Mora.TemporalQueue.Behaviour do
  @moduledoc """
  This module defines the behaviour of the temporal queue.
  """
  @callback enqueue(Mora.Model.Event.t(), map()) :: :ok | {:error, any()}
end
