defmodule Mora.Service.EventsBehaviour do
  @moduledoc """
  This module contains the behaviour for the events service.
  """
  @callback(process_events([Mora.Model.Event.t()]) :: :ok, {:error, error})
end
