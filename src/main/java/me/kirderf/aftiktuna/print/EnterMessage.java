package me.kirderf.aftiktuna.print;

import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;

import java.util.List;
import java.util.Optional;
import java.util.stream.Stream;

public final class EnterMessage implements Message {
	private final List<Aftik> aftiks;
	private final Door door;
	
	public EnterMessage(Aftik aftik, Door door) {
		this(List.of(aftik), door);
	}
	
	private EnterMessage(List<Aftik> aftiks, Door door) {
		this.aftiks = aftiks;
		this.door = door;
	}
	
	@Override
	public String getText() {
		return "%s entered the %s into a new area.".formatted(asList(aftiks), door.getType().getCategoryName());
	}
	
	private static String asList(List<Aftik> aftiks) {
		StringBuilder builder = new StringBuilder();
		builder.append(aftiks.get(0).getName());
		for (int i = 1; i < aftiks.size(); i++) {
			if (i < aftiks.size() - 1)
				builder.append(", ");
			else
				builder.append(" and ");
			builder.append(aftiks.get(i).getName());
		}
		return builder.toString();
	}
	
	@Override
	public Optional<Message> tryCombine(Message other) {
		if (other instanceof EnterMessage otherEnter && otherEnter.door == this.door) {
			return Optional.of(new EnterMessage(Stream.concat(this.aftiks.stream(), otherEnter.aftiks.stream()).toList(), door));
		} else
			return Optional.empty();
	}
}
