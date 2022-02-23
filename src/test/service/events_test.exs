defmodule Mora.Test.Service.Events do
  @moduledoc """
  This module contains the tests for the events service.
  """
  use ExUnit.Case, async: true
  alias Mora.Service.Events
  alias Mora.Support.Generator
  import Mox
  import ExUnit.CaptureLog
  setup :verify_on_exit!

  describe "Events Service" do
    test "process_events/1 should send events to database and notify queues" do
      expect(Mora.DatabaseMock, :save, 1, fn _ -> :ok end)
      expect(Mora.TemporalQueue.ManagerMock, :notify, 1, fn _ -> :ok end)

      Events.process_events([Generator.get_random_event()])
    end

    test "process_events/1 should not notify event if database save is not successful" do
      expect(Mora.DatabaseMock, :save, 1, fn _ -> {:error, :save} end)
      expect(Mora.TemporalQueue.ManagerMock, :notify, 0, fn _ -> :ok end)
      expect(Mora.DatabaseMock, :delete, 0, fn _ -> :ok end)
      expect(Mora.TemporalQueue.ManagerMock, :unschedule, 0, fn _ -> :ok end)

      Events.process_events([Generator.get_random_event()])
    end

    test "process_events/1 should delete event from database if notify not successful" do
      expect(Mora.DatabaseMock, :save, 1, fn _ -> :ok end)
      expect(Mora.TemporalQueue.ManagerMock, :notify, fn _ -> {:error, {:notify}} end)
      expect(Mora.DatabaseMock, :delete, 1, fn _ -> :ok end)
      expect(Mora.TemporalQueue.ManagerMock, :unschedule, 0, fn _ -> :ok end)

      Events.process_events([Generator.get_random_event()])
    end
  end
end
