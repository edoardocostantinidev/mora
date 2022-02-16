Mox.defmock(Mora.TemporalQueue.ManagerMock, for: Mora.TemporalQueue.ManagerBehaviour)
Mox.defmock(Mora.DatabaseMock, for: Mora.DatabaseBehaviour)
Mox.defmock(Mora.Service.EventsMock, for: Mora.Service.EventsBehaviour)

Application.put_env(:mora, :temporal_queue_manager, Mora.TemporalQueue.ManagerMock)
Application.put_env(:mora, :database, Mora.DatabaseMock)
Application.put_env(:mora, :events_service, Mora.Service.EventsMock)

ExUnit.start()
