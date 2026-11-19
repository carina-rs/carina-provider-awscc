Post-migration identity pins for #572, not historical baseline data.

catalog.txt pins every identity in AwsccType::ALL, StringIdentity::ALL and the
new AWSCC-owned ScalarType::ALL, including their declarations. This collection
was assembled for #572 from catalog rows and helper declarations; it was not a
catalog-only capture from base a1bed43. The unused CloudFront HostedZoneId row
has been removed. aws.kms.Key.Id itself existed in the base StringIdentity and
catalog constructor; ScalarType as an owned collection was introduced in #572.

resolver.txt pins the accepted identity of the owned KMS resolver. It is an
explicit head pin, not an observation of the base resolver's implementation.

Both files carry awscc names and are compared exactly, without provider mapping.
The published schema/callback/completion migration baseline remains separately
in identity_inventory_before_572.
