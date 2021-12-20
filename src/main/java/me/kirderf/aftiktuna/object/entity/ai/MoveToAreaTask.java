package me.kirderf.aftiktuna.object.entity.ai;

import me.kirderf.aftiktuna.action.EnterDoorAction;
import me.kirderf.aftiktuna.location.Area;
import me.kirderf.aftiktuna.object.Identifier;
import me.kirderf.aftiktuna.object.door.Door;
import me.kirderf.aftiktuna.object.entity.Aftik;
import me.kirderf.aftiktuna.print.ActionPrinter;

import java.util.Optional;

public final class MoveToAreaTask extends Task {
	private final Identifier<Area> areaId;
	
	public MoveToAreaTask(Area area) {
		this.areaId = area.getId();
	}
	
	@Override
	public Status prepare(Aftik aftik) {
		if (!aftik.getArea().getId().equals(areaId)) {
			if (findPathTowardsArea(aftik, areaId).map(door -> !aftik.isAccessible(door.getPosition(), true)).orElse(true))
				return Status.REMOVE;
			else
				return Status.KEEP;
		} else
			return Status.REMOVE;
	}
	
	@Override
	public Status performAction(Aftik aftik, ActionPrinter out) {
		if (!aftik.getArea().getId().equals(areaId)) {
			return tryGoToArea(aftik, areaId, out);
		} else {
			return Status.REMOVE;
		}
	}
	
	static Status tryGoToArea(Aftik aftik, Identifier<Area> id, ActionPrinter out) {
		Optional<Door> optional = findPathTowardsArea(aftik, id);
		if (optional.isPresent()) {
			Door door = optional.get();
			
			EnterDoorAction.Result result = EnterDoorAction.moveAndEnter(aftik, door, out);
			
			return result.success() ? Status.KEEP : Status.REMOVE;
		} else {
			out.printFor(aftik, "%s is unable to proceed to their destination.", aftik.getName());
			return Status.REMOVE;
		}
	}
	
	public static Optional<Door> findPathTowardsArea(Aftik aftik, Area area) {
		return findPathTowardsArea(aftik, area.getId());
	}
	
	public static Optional<Door> findPathTowardsArea(Aftik aftik, Identifier<Area> id) {
		return aftik.getMind().getMemory().findDoorTowards(aftik.getArea(), id);
	}
}