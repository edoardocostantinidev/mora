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
        {:error, e} ->
          Logger.error("Error while processing event: #{inspect(e)}")
          delete_from_database(event)
          unschedule(event)
      end
    end)
  end

  defp delete_from_database(event) do
    get_database().delete(event.id)
  end

  defp save_to_database(event) do
    get_database().save(event)
  end

  defp unschedule(event) do
    get_queue_manager().unschedule(event)
  end

  defp notify(event) do
    get_queue_manager().notify(event)
  end

  defp get_database(), do: Application.get_env(:mora, :database, Mora.Database.Mnesia)

  defp get_queue_manager(),
    do: Application.get_env(:mora, :temporal_queue_manager, Mora.TemporalQueue.Manager)
end
