import * as beet from '@metaplex-foundation/beet';
export enum EndSettingType {
  Date,
  Amount,
}
export const endSettingTypeBeet = beet.fixedScalarEnum(EndSettingType) as beet.FixedSizeBeet<
  EndSettingType,
  EndSettingType
>;
