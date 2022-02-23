defmodule Mora.TemporalQueue.ManagerBehaviour do
  @moduledoc """
  This module defines the behaviour of the temporal queue manager.
  """
  @callback notify(Mora.Model.Event.t()) :: :ok
  @callback unschedule(Mora.Model.Event.t()) :: :ok
end
