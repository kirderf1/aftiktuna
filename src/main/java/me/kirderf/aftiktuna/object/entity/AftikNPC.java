package me.kirderf.aftiktuna.object.entity;

import me.kirderf.aftiktuna.Crew;
import me.kirderf.aftiktuna.location.GameObject;
import me.kirderf.aftiktuna.object.type.ObjectTypes;
import me.kirderf.aftiktuna.util.OptionalFunction;

public final class AftikNPC extends GameObject {
	public static final OptionalFunction<GameObject, AftikNPC> CAST = OptionalFunction.cast(AftikNPC.class);
	
	private final String name;
	private final Stats stats;
	
	public AftikNPC(String name, Stats stats) {
		super(ObjectTypes.AFTIK, 15);
		this.name = name;
		this.stats = stats;
	}
	
	public Aftik createAftikForCrew(Crew crew) {
		Aftik aftik = new Aftik(name, stats, crew);
		this.getArea().addObject(aftik, this.getPosition());
		this.remove();
		return aftik;
	}
}