defmodule Mora.Database do
  @doc """
  saves an event
  """
  @callback save(Mora.Model.Event.t()) :: :ok | {:error, String.t()}

  @doc """
  retrieves n events from a given point in time
  """
  @callback get_from(list()) ::
              {:ok, [Mora.Model.Event.t()]} | {:error, String.t()}
end
