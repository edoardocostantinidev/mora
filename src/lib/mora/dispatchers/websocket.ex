defmodule Mora.Dispatchers.Websocket do
  @moduledoc """
  This module provides a websocket dispatcher.
  """

  @behaviour Mora.Dispatcher
  @behaviour Mora.CommonBehaviour.PgItem

  @pg_name "dispatchers:websocket"
  @pg_system_name "system:dispatchers:websocket"

  require Logger
  use GenServer
  alias Mora.Model.Event

  @spec start_link(any) :: :ignore | {:error, any} | {:ok, pid}
  def start_link(_opts) do
    GenServer.start_link(__MODULE__, :ok, name: __MODULE__)
  end

  @doc """
  starts up a websocket dispatcher.
  """
  def init(_) do
    :ok = :pg.join(pg_name(""), self())
    {:ok, {}}
  end

  def handle_cast({:dispatch, event}, state) do
    dispatch(event)
    {:noreply, state}
  end

  @spec dispatch(Event.t()) :: {:ok}
  def dispatch(event) do
    event.category
    |> Mora.Api.SocketHandler.Event.pg_name()
    |> :pg.get_members()
    |> Enum.each(&Process.send(&1, event, []))

    {:ok}
  end

  def pg_name(_), do: @pg_name
  def pg_system_name(), do: @pg_system_name
end
