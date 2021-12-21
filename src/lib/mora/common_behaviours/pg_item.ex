defmodule Mora.CommonBehaviour.PgItem do
  @moduledoc """
  Common behaviours for all pg registered modules.
  """

  @doc """
  Gets the pg item's name.
  """
  @callback pg_name(category :: String.t()) :: String.t()

  @doc """
  Gets the pg item system name. used mainly for telemetry.
  """
  @callback pg_system_name() :: String.t()
end
