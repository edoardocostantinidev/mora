defmodule Mora.TemporalQueue.Manager do
  @moduledoc """
  This module provides a queue manager for the Mora.TemporalQueue module.
  """
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
    GenServer.cast(__MODULE__, {:notify, event})
  end

  def handle_cast({:notify, event}, state) do
    event.category
    |> Mora.TemporalQueue.pg_name()
    |> :pg.get_members()
    |> case do
      [] ->
        :pg.get_members(@pg_name)
        |> Enum.filter(fn p -> p != self() end)
        |> Enum.each(fn pid -> GenServer.call(pid, {:spawn_and_notify, event}) end)

        spawn_and_notify(event)

      queues ->
        Enum.each(queues, &GenServer.cast(&1, {:notify, event}))
    end

    {:noreply, state}
  end

  def handle_call({:spawn_and_notify, event}, _, state) do
    spawn_and_notify(event)
    {:noreply, state}
  end

  defp spawn_and_notify(event) do
    {:ok, queue} = Mora.TemporalQueue.DynamicSupervisor.start_temporal_queue(event.category)
    GenServer.cast(queue, {:notify, event})
  end

  def pg_name(_), do: @pg_name
  def pg_system_name(), do: @pg_system_name
end
