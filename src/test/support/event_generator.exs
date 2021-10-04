defmodule Mora.Events.Generator do
  def get_random_event do
    get_random_event(1_682_235_256_700, 1_692_255_256_733)
  end

  def get_random_event(fire_at_min, fire_at_max) do
    event =
      struct(Mora.Event,
        fireAt: :rand.uniform(fire_at_max - fire_at_min) + fire_at_min,
        createdAt: :rand.uniform(fire_at_max - fire_at_min) + fire_at_min,
        category: :crypto.strong_rand_bytes(10)
      )

    event_hash = :erlang.phash2(event)
    Map.put(event, :id, "#{event.createdAt}-#{event.fireAt}-#{event_hash}")
  end
end
