defmodule Mora.TemporalQueue.DynamicSupervisor do
  use DynamicSupervisor
  @supervisor_pg_name "supervisors:temporalqueue"

  def start_link(arg) do
    DynamicSupervisor.start_link(__MODULE__, arg, name: __MODULE__)
  end

  def init(arg) do
    DynamicSupervisor.init(arg)
  end

  def handle_cast({:notify, event}) do
    :pg.get_local_members("#{event.category}:temporalqueue")
    |> case do
      [] ->
        {:ok, queue} = start_temporal_queue(event.category)
        GenServer.cast(queue, {:notify, event})

      queues ->
        Enum.each(queues, fn queue -> GenServer.cast(queue, {:notify, event}) end)
    end
  end

  def start_temporal_queue(category) do
    spec = {Mora.TemporalQueue.Priority, %{category: category}}
    DynamicSupervisor.start_child(__MODULE__, spec)
  end

  def notify(event) do
    :pg.get_members(@supervisor_pg_name)
    |> Enum.each(fn member ->
      GenServer.cast(member, {:notify, event})
    end)
  end
end
