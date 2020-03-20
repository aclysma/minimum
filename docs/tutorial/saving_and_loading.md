
# Saving and Loading

Starting from the result of dev_environment_setup.md, lets try to save the world to disk.

First, we need to change our components

```
#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Default)]
#[uuid = "8bf67228-f96c-4649-b306-ecd107190000"]
pub struct PositionComponent {
    #[serde_diff(opaque)]
    pub position: Vec2,
}

legion_prefab::register_component_type!(PositionComponent);

#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Default)]
#[uuid = "8bf67228-f96c-4649-b306-ecd107190001"]
pub struct VelocityComponent {
    #[serde_diff(opaque)]
    pub position: Vec2,
}

legion_prefab::register_component_type!(VelocityComponent);

#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Default)]
#[uuid = "8bf67228-f96c-4649-b306-ecd107190002"]
pub struct AccelerationComponent {
    #[serde_diff(opaque)]
    pub position: Vec2,
}

legion_prefab::register_component_type!(AccelerationComponent);

#[derive(TypeUuid, Clone, Serialize, Deserialize, SerdeDiff, Debug, Default)]
#[uuid = "8bf67228-f96c-4649-b306-ecd107190003"]
pub struct GravityComponent {
    #[serde_diff(opaque)]
    pub position: Vec2,
}

legion_prefab::register_component_type!(GravityComponent);
```

