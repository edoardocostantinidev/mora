defmodule Moradb.Events.Database do
  @doc """
  saves an event
  """
  @callback save(Moradb.Event.t()) :: {:ok} | {:error, String.t()}

  @doc """
  retrieves n events from a given point in time
  """
  @callback get_from(list()) ::
              {:ok, [Moradb.Event.t()]} | {:error, String.t()}
end
