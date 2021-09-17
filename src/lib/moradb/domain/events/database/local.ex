defmodule Moradb.Events.Database.Local do
  @behaviour Moradb.Events.Database
  require Logger

  def save(event) do
    Logger.info("saving #{event.id} locally")
    {:error, "not implemented"}
  end

  def get_from(timestamp, limit \\ 100) do
    Logger.info("getting #{limit} from #{timestamp} onwards")
    {:error, "not implemented"}
  end
end
