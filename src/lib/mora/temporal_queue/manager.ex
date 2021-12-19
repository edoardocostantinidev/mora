defmodule Mora.TemporalQueue.Manager do
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
    IO.inspect(event)
    GenServer.cast(__MODULE__, {:notify, event})
  end

  def handle_cast({:notify, event}, state) do
    :pg.get_members("temporal_queues:#{event.category}")
    |> IO.inspect(label: :queues)
    |> case do
      [] ->
        :pg.get_members(@pg_name)
        |> IO.inspect(label: :managers)
        |> Enum.each(fn manager -> GenServer.cast(manager, {:spawn_and_notify, event}) end)

      queues ->
        Enum.each(queues, fn queue -> GenServer.cast(queue, {:notify, event}) end)
    end

    {:noreply, state}
  end

  def handle_cast({:spawn_and_notify, event}, state) do
    {:ok, queue} = Mora.TemporalQueue.DynamicSupervisor.start_temporal_queue(event.category)
    GenServer.cast(queue, {:notify, event})
    {:noreply, state}
  end
end
