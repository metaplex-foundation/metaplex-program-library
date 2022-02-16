import * as beet from '@metaplex-foundation/beet';
export enum EndSettingType {
  Date,
  Amount,
}
export const endSettingTypeEnum = beet.fixedScalarEnum(EndSettingType) as beet.FixedSizeBeet<
  EndSettingType,
  EndSettingType
>;
