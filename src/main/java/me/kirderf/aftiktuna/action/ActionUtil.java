package me.kirderf.aftiktuna.action;

import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.location.Position;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.util.OptionalFunction;

import java.util.Optional;
import java.util.function.IntSupplier;
import java.util.function.ToIntFunction;

public final class ActionUtil {
	static <T extends GameObject> int searchForAccessible(InputActionContext context, Aftik aftik,
														  OptionalFunction<GameObject, T> mapper, boolean exactPos,
														  ToIntFunction<T> onSuccess, IntSupplier onNoMatch) {
		Optional<T> optionalDoor = aftik.findNearest(mapper, exactPos);
		if (optionalDoor.isPresent()) {
			T object = optionalDoor.get();
			Position pos = exactPos ? object.getPosition()
					: object.getPosition().getPosTowards(aftik.getCoord());
			
			return ifAccessible(context, aftik, pos, () -> onSuccess.applyAsInt(object));
		} else {
			return onNoMatch.getAsInt();
		}
	}
	
	static int ifAccessible(InputActionContext context, Aftik aftik, Position pos, IntSupplier onSuccess) {
		Optional<GameObject> blocking = aftik.findBlockingTo(pos.coord());
		if (blocking.isEmpty()) {
			return onSuccess.getAsInt();
		} else {
			return context.printNoAction(createBlockingMessage(blocking.get()));
		}
	}
	
	public static String createBlockingMessage(GameObject blocking) {
		return "%s is blocking the way.".formatted(blocking.getDisplayName(true, true));
	}
	
	public static String condition(String text, boolean b) {
		if (b)
			return text.replaceAll("[\\[\\]]", "");
		else return text.replaceAll("\\[.*]", "");
	}
}