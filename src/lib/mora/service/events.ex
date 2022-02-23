defmodule Mora.Service.Events do
  @moduledoc """
  This module provides the event service.
  """
  @behaviour Mora.Service.EventsBehaviour
  require Logger

  def process_events(events) do
    events
    |> Enum.map(fn event ->
      event_hash = :erlang.phash2(event)
      Map.put(event, :id, "#{event.createdAt}-#{event.fireAt}-#{event_hash}")
    end)
    |> Enum.each(fn event ->
      with :ok <- save_to_database(event),
           :ok <- notify(event) do
        :ok
      else
        {:error, :notify, _} ->
          delete_from_database(event)

        _ ->
          nil
      end
    end)
  end

  defp delete_from_database(event) do
    get_database().delete(event.id)
    |> case do
      {:error, e} -> {:error, :delete, e}
      :ok -> :ok
    end
  end

  defp save_to_database(event) do
    get_database().save(event)
    |> case do
      {:error, e} -> {:error, :save, e}
      :ok -> :ok
    end
  end

  defp notify(event) do
    get_queue_manager().notify(event)
    |> case do
      {:error, e} -> {:error, :notify, e}
      :ok -> :ok
    end
  end

  defp get_database(), do: Application.get_env(:mora, :database, Mora.Database.Mnesia)

  defp get_queue_manager(),
    do: Application.get_env(:mora, :temporal_queue_manager, Mora.TemporalQueue.Manager)
end
