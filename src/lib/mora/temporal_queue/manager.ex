defmodule Mora.TemporalQueue.Manager do
  @moduledoc """
  This module provides a queue manager for the Mora.TemporalQueue module.
  """
  @behaviour Mora.TemporalQueue.ManagerBehaviour
  use GenServer
  @pg_name "managers:temporal_queue"
  @pg_system_name "system:managers"
  def start_link(_) do
    GenServer.start_link(__MODULE__, :ok, name: __MODULE__)
  end

  def init(:ok) do
    :pg.join(@pg_name, self())
    :pg.join(@pg_system_name, self())
    {:ok, %{}}
  end

  def notify(event) do
    GenServer.call(__MODULE__, {:notify, event})
  end

  def unschedule(event) do
    GenServer.call(__MODULE__, {:unschedule, event})
  end

  def handle_call({:unschedule, %{category: category, id: binary}}, _, state) do
    category
    |> Mora.TemporalQueue.Server.pg_name()
    |> :pg.get_members()
    |> case do
      queues ->
        Enum.each(queues, &GenServer.call(&1, {:unschedule, id: binary}))
    end

    {:reply, :ok, state}
  end

  def handle_call({:notify, event}, _, state) do
    event.category
    |> Mora.TemporalQueue.Server.pg_name()
    |> :pg.get_members()
    |> case do
      [] ->
        :pg.get_members(@pg_name)
        |> Enum.filter(fn p -> p != self() end)
        |> Enum.each(fn pid -> GenServer.call(pid, {:spawn_and_notify, event}) end)

        spawn_and_notify(event)

      queues ->
        Enum.each(queues, &GenServer.call(&1, {:notify, event}))
    end

    {:reply, :ok, state}
  end

  def handle_call({:spawn_and_notify, event}, _, state) do
    spawn_and_notify(event)
    {:reply, :ok, state}
  end

  defp spawn_and_notify(event) do
    {:ok, queue} = Mora.TemporalQueue.DynamicSupervisor.start_temporal_queue(event.category)
    GenServer.call(queue, {:notify, event})
  end

  def pg_name(_), do: @pg_name
  def pg_system_name(), do: @pg_system_name
end
