defmodule Mora.TemporalQueue.DynamicSupervisor do
  use DynamicSupervisor

  require Logger
  @pg_name "supervisors:temporalqueue"
  @pg_system_name "system:supervisors"
  def start_link(_) do
    DynamicSupervisor.start_link(__MODULE__, :ok, name: __MODULE__)
  end

  def init(:ok) do
    :pg.join(@pg_name, self())
    :pg.join(@pg_system_name, self())
    Logger.debug("Supervisor joined pg with: #{inspect(self())}")
    DynamicSupervisor.init(strategy: :one_for_one)
  end

  def start_temporal_queue(category) do
    spec = {Mora.TemporalQueue, %{category: category}}
    DynamicSupervisor.start_child(__MODULE__, spec)
  end
end
